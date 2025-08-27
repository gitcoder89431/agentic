use crate::ui::app::{AgentStatus, AppMode};
use agentic_core::theme::{Element, Theme};
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Frame, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub struct AutocompleteParams<'a> {
    pub show: bool,
    pub commands: &'a [(String, String)],
    pub selected_index: usize,
}

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
    autocomplete: AutocompleteParams,
    ruixen_emoji: &str,
) {
    let chat_block = Block::new()
        .borders(Borders::ALL)
        .title(format!(" {} ", ruixen_emoji))
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

            // Show autocomplete dropdown if needed
            if autocomplete.show && !autocomplete.commands.is_empty() {
                let dropdown_height = (autocomplete.commands.len() as u16).clamp(1, 6) + 2; // Add 2 for borders
                let dropdown_width = 50u16.min(inner_area.width - 4); // Make wider and ensure space for borders

                // Position dropdown just above the footer bar (bottom of chat area)
                let dropdown_area = Rect::new(
                    inner_area.x + 2,
                    inner_area.y + inner_area.height.saturating_sub(dropdown_height + 1),
                    dropdown_width,
                    dropdown_height,
                );

                render_autocomplete_dropdown(
                    frame,
                    dropdown_area,
                    theme,
                    autocomplete.commands,
                    autocomplete.selected_index,
                );
            }
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

            // Status-based message - show one error at a time, prioritizing local > cloud > api key
            let (status_text, status_style) = match agent_status {
                AgentStatus::LocalEndpointError => (
                    "Local not ready - see [S]ettings",
                    theme.ratatui_style(Element::Warning),
                ),
                AgentStatus::CheckLocalModel => (
                    "Local not ready - see [S]ettings",
                    theme.ratatui_style(Element::Warning),
                ),
                AgentStatus::CloudEndpointError => (
                    "Cloud not ready - see [S]ettings",
                    theme.ratatui_style(Element::Warning),
                ),
                AgentStatus::CheckCloudModel => (
                    "Cloud not ready - see [S]ettings",
                    theme.ratatui_style(Element::Warning),
                ),
                AgentStatus::CheckApiKey => (
                    "API Key not ready - see [S]ettings",
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
                    "Press [ENTER] to start Ruixen",
                    theme.ratatui_style(Element::Accent),
                ),
            };

            let status_paragraph = Paragraph::new(status_text)
                .alignment(Alignment::Center)
                .style(status_style);

            frame.render_widget(status_paragraph, vertical_chunks[3]);
        }
    }
}

fn render_autocomplete_dropdown(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    commands: &[(String, String)],
    selected_index: usize,
) {
    let items: Vec<ListItem> = commands
        .iter()
        .enumerate()
        .map(|(i, (cmd, desc))| {
            let (cmd_style, desc_style) = if i == selected_index {
                // Selected item: use accent color for command, normal for description
                (theme.ratatui_style(Element::Accent), theme.text_style())
            } else {
                // Non-selected: normal text for both
                (theme.text_style(), theme.ratatui_style(Element::Inactive))
            };

            let line = Line::from(vec![
                Span::styled(cmd.clone(), cmd_style),
                Span::raw(" - "),
                Span::styled(desc.clone(), desc_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Commands")
            .style(theme.ratatui_style(Element::Active)),
    );

    frame.render_widget(list, area);
}
