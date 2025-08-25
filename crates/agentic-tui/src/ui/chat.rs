use agentic_core::theme::{Element, Theme};
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Frame, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
};

const LOGO: &str = r#"
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
const LOGO_HEIGHT: u16 = 15;
const TEXT_HEIGHT: u16 = 1;
const GAP_HEIGHT: u16 = 1;
const TOTAL_HEIGHT: u16 = LOGO_HEIGHT + GAP_HEIGHT + TEXT_HEIGHT;

pub fn render_chat(frame: &mut Frame, area: Rect, theme: &Theme) {
    let chat_block = Block::new()
        .borders(Borders::ALL)
        .title(" 🤨 🔍 💡 ")
        .style(theme.ratatui_style(Element::Text));

    let inner_area = chat_block.inner(area);

    frame.render_widget(chat_block, area);

    let top_padding = (inner_area.height.saturating_sub(TOTAL_HEIGHT)) / 2;

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_padding),
            Constraint::Length(LOGO_HEIGHT),
            Constraint::Length(GAP_HEIGHT),
            Constraint::Length(TEXT_HEIGHT),
            Constraint::Min(0),
        ])
        .split(inner_area);

    let logo_paragraph = Paragraph::new(LOGO)
        .alignment(Alignment::Center)
        .style(theme.ratatui_style(Element::Text));

    frame.render_widget(logo_paragraph, vertical_chunks[1]);

    let start_text = "Press [ENTER] to Start Ruixen";
    let start_paragraph = Paragraph::new(start_text)
        .alignment(Alignment::Center)
        .style(theme.ratatui_style(Element::Text));

    frame.render_widget(start_paragraph, vertical_chunks[3]);
}
