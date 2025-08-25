use crate::ui::app::{AgentStatus, AppMode};
use agentic_core::theme::{Element, Theme};
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Frame, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
};

const MAIN_LOGO: &str = r#"
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║      █████╗  ██████╗ ███████╗███╗   ██╗████████╗██╗ ██████╗   ║
    ║     ██╔══██╗██╔════╝ ██╔════╝████╗  ██║╚══██╔══╝██║██╔════╝   ║
    ║     ███████║██║  ███╗█████╗  ██╔██╗ ██║   ██║   ██║██║        ║
    ║     ██╔══██║██║   ██║██╔══╝  ██║╚██╗██║   ██║   ██║██║        ║
    ║     ██║  ██║╚██████╔╝███████╗██║ ╚████║   ██║   ██║╚██████╗   ║
    ║     ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝ ╚═════╝   ║
    ║                                                               ║
    ║                    The Agent you work WITH                    ║
    ║                                                               ║
    ║              AI Model Orchestrator & Agent Framework          ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝
"#;

const SPIRAL_GALAXY: &str = r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀⠀⠀⠀⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠂⠀⠀⠀⢀⠀⠀⠀⠀⠀⠀⠀⠐⠀⠀⠂⠀⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠄⠀⠠⠀⣀⢄⡠⣔⢬⠃⠵⠯⠂⡠⢦⠀⠀⡀⠄⠄⠀⠀⠀⠀⡀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⢀⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠀⢀⡔⣌⣔⣏⠋⠄⡪⡹⡝⠋⡛⣃⠃⠆⡅⡄⠂⠀⠐⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠐
⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡴⡰⣷⠾⡃⡌⢍⠉⠈⠈⠀⠈⠀⠉⠂⠀⠀⠀⠀⠄⠀⠀⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠂⠀⠀⠀⠀⠀⡼⣷⣽⡋⠉⠁⠀⠀⠀⠀⠀⠐⠀⠀⠀⡀⠄⠀⠀⠐⠈⠀⢀⠄⢈⠀⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⢄⡔⣿⣯⠋⠂⠀⠈⠀⠀⢄⣠⡌⡤⡆⣄⠦⣧⡒⠠⠄⢐⡀⠀⠀⠀⠀⠀⡠⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⢀⠀⠀⠀⠀⠈⠄⠑⣯⡟⡗⠳⠀⠀⠀⠀⣀⡨⣿⡯⡋⡍⡉⢙⠮⠹⢗⠟⡓⢣⢥⢂⣁⠄⡀⠃⠀⠀⡀⠀⠀⠀⠀⠀⠀⠀⠈
⠀⠀⠀⠀⠀⠀⠀⠀⠀⣣⣿⠿⡧⠊⠀⠀⠁⡀⣵⡿⠞⡁⠔⠅⠀⠀⠁⠀⠀⠉⠉⠨⠽⢕⠧⣜⠅⡺⡇⠄⠠⠄⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢈⠀⠂⠀⠂⡸⣾⣷⠠⠀⡀⠀⣠⣚⡕⡁⠁⠀⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠋⠕⡥⠭⣦⢈⡄⠀⠁⠀⠀⠀⠁⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠐⠀⢦⡧⡇⡀⠀⡂⠀⠂⣝⣃⠁⠠⠁⠀⠀⠀⠀⠀⠀⡀⠀⠀⠀⠀⠀⠀⠀⠈⢣⡕⡷⡕⠀⠱⠑⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠐⠌⣻⡯⡃⡀⡀⠀⠀⠁⠾⡕⠅⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡽⠭⣻⠢⠀⠀⠀⠀⠀⠀⠀⠈
⠀⠀⠀⠁⠀⠈⠠⠀⠠⣿⡣⡀⠀⠀⠀⠀⠐⢿⡅⡋⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⠀⠀⠠⡿⠵⠀⠀⠀⠀⠄⢀⠀⠀⠀
⠀⠀⠀⡀⠀⠀⠐⠀⠤⣾⡏⡒⠄⠀⠀⠀⡈⠉⠟⠿⣷⣯⠦⠆⠤⠠⢀⣀⠄⠀⠀⢀⠀⠁⠀⠀⠀⠀⠆⣜⣷⠀⠃⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⡘⠀⢘⣷⠓⠀⡀⠀⠀⠁⠀⠀⠉⠈⢚⡋⡲⡧⠍⣨⣀⠄⠀⠀⠀⠀⠀⠀⠀⣀⢡⣹⣞⡗⠉⠂⠠⠀⠀⠀⠀⠁⠀
⠀⠀⠀⠀⠀⠀⠀⢠⠀⢼⣽⡷⡄⠈⠂⠁⠀⠀⠀⠀⠁⠄⠐⠊⡣⢆⠀⢒⠄⠀⠀⠀⠀⠀⠀⠈⡁⢅⣏⡟⢥⠀⠀⠀⠀⠀⠀⠀⠀⠂
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠏⡻⣿⣃⡔⠀⠂⡀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠀⠀⠀⠀⡀⠀⠠⢰⠂⠥⠶⡿⡛⡁⠂⠀⠂⡀⠄⠀⠀⠈⠀⠀
⠀⠀⠉⠀⠀⠀⠀⠀⠁⠁⡩⣽⠧⢁⡌⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⢀⡀⠠⠆⢨⣷⢭⡙⠨⠠⡀⠁⠐⠀⠀⠈⠀⠁⠀⢄⢁
⠀⠀⠀⠀⠀⠀⠀⠀⠄⠁⠡⡛⢪⠝⣦⣥⡢⠀⠀⠀⡀⠀⠀⡀⠀⠀⡔⠔⠔⢀⣥⣡⢆⡜⣟⠌⡀⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠀
⠀⠄⠀⠀⠀⠀⠀⠀⠀⠀⠉⠝⣅⡀⣴⣽⡽⡇⢧⡮⢂⣀⡶⣊⢪⢙⠘⣽⡕⣋⠥⡏⠅⡁⠀⠀⠁⠀⠀⠢⡀⠀⠀⠀⠀⠁⠀⠀⠀⠐
⠀⠀⠀⠀⠀⠀⠀⠀⠁⠠⠀⠀⠈⠌⠀⠑⢉⠻⠋⠳⢥⠧⡤⠅⠂⠏⢁⠼⡯⠋⠈⠀⠀⠀⠀⠀⠠⠀⠀⠀⠀⠀⠀⠀⠀⠔⠀⠂⠀⠀
⠀⠀⠀⠀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠢⠁⣑⠎⠄⡠⡆⡰⡁⠸⠋⠉⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠨⠐⠈⠀⠀⠀⠀⠀⠄⠀⠄⠀
⠀⠀⠁⡀⠀⠀⠀⠠⠀⠀⠀⠀⠀⠂⠀⠀⠈⠁⠁⠈⠀⠁⠀⠄⠀⡄⠀⠀⠀⠀⠀⠑⠀⠀⠂⠀⠀⠄⠀⠀⠀⠄⠄⠂⠁⠀⠀⠀⠀⠀
⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠠⠂⠀⠀⠀⠄⠀⠀⠀⠀⠐⠐⠀⠀⠀⠀⠀⠀⠀
"#;
const MAIN_LOGO_HEIGHT: u16 = 15;
const SPIRAL_GALAXY_HEIGHT: u16 = 25;
const TEXT_HEIGHT: u16 = 1;
const GAP_HEIGHT: u16 = 1;
const MAIN_TOTAL_HEIGHT: u16 = MAIN_LOGO_HEIGHT + GAP_HEIGHT + TEXT_HEIGHT;
const SPIRAL_TOTAL_HEIGHT: u16 = SPIRAL_GALAXY_HEIGHT;

pub fn render_chat(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    mode: AppMode,
    chat_input: &str,
    agent_status: AgentStatus,
) {
    let chat_block = Block::new()
        .borders(Borders::ALL)
        .title(" 🤨 🔍 💡 ")
        .style(theme.ratatui_style(Element::Text));

    let inner_area = chat_block.inner(area);
    frame.render_widget(chat_block, area);

    match mode {
        AppMode::Chat if chat_input.is_empty() => {
            // Show spiral galaxy screensaver when in chat mode but no input yet
            let top_padding = (inner_area.height.saturating_sub(SPIRAL_TOTAL_HEIGHT)) / 2;

            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(top_padding),
                    Constraint::Length(SPIRAL_GALAXY_HEIGHT),
                    Constraint::Min(0),
                ])
                .split(inner_area);

            let spiral_paragraph = Paragraph::new(SPIRAL_GALAXY)
                .alignment(Alignment::Center)
                .style(theme.ratatui_style(Element::Inactive));

            frame.render_widget(spiral_paragraph, vertical_chunks[1]);
        }
        AppMode::Chat => {
            // Clean canvas when user is typing - completely empty
            // The spiral galaxy has disappeared, leaving pure focus space
        }
        _ => {
            // Normal mode - show main logo with status-based message
            let top_padding = (inner_area.height.saturating_sub(MAIN_TOTAL_HEIGHT)) / 2;

            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(top_padding),
                    Constraint::Length(MAIN_LOGO_HEIGHT),
                    Constraint::Length(GAP_HEIGHT),
                    Constraint::Length(TEXT_HEIGHT),
                    Constraint::Min(0),
                ])
                .split(inner_area);

            let logo_paragraph = Paragraph::new(MAIN_LOGO)
                .alignment(Alignment::Center)
                .style(theme.ratatui_style(Element::Text));

            frame.render_widget(logo_paragraph, vertical_chunks[1]);

            // Status-based message
            let (status_text, status_style) = match agent_status {
                AgentStatus::Ready => (
                    "Press [ENTER] to Start Ruixen",
                    theme.ratatui_style(Element::Accent),
                ),
                AgentStatus::LocalEndpointError => (
                    "⚠️  Local endpoint error - Check settings [S]",
                    theme.ratatui_style(Element::Warning),
                ),
                AgentStatus::CloudEndpointError => (
                    "⚠️  Cloud endpoint error - Check settings [S]",
                    theme.ratatui_style(Element::Warning),
                ),
                AgentStatus::CheckLocalModel => (
                    "⚠️  Local model not configured - Check settings [S]",
                    theme.ratatui_style(Element::Warning),
                ),
                AgentStatus::CheckCloudModel => (
                    "⚠️  Cloud model not configured - Check settings [S]",
                    theme.ratatui_style(Element::Warning),
                ),
                AgentStatus::CheckApiKey => (
                    "⚠️  API key not configured - Check settings [S]",
                    theme.ratatui_style(Element::Warning),
                ),
                AgentStatus::ValidatingLocal => (
                    "🔄 Validating local endpoint...",
                    theme.ratatui_style(Element::Info),
                ),
                AgentStatus::ValidatingCloud => (
                    "🔄 Validating cloud endpoint...",
                    theme.ratatui_style(Element::Info),
                ),
                _ => (
                    "Press [ENTER] when local and cloud models are ready",
                    theme.ratatui_style(Element::Inactive),
                ),
            };

            let status_paragraph = Paragraph::new(status_text)
                .alignment(Alignment::Center)
                .style(status_style);

            frame.render_widget(status_paragraph, vertical_chunks[3]);
        }
    }
}
