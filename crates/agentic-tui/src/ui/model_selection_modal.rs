use agentic_core::theme::{Element, Theme};
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Frame, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub struct ModelSelectionParams<'a> {
    pub theme: &'a Theme,
    pub title: &'a str,
    pub models: &'a [(String, String)], // (name, info)
    pub selected_index: usize,
    pub current_page: usize,
    pub models_per_page: usize,
}

pub fn render_model_selection_modal(frame: &mut Frame, area: Rect, params: ModelSelectionParams) {
    let block = Block::new()
        .title(params.title)
        .borders(Borders::ALL)
        .style(params.theme.ratatui_style(Element::Text));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if params.models.is_empty() {
        // Show loading or error message
        let loading_text = "Loading available models...";
        let loading_paragraph = Paragraph::new(loading_text)
            .alignment(Alignment::Center)
            .style(params.theme.ratatui_style(Element::Inactive));

        frame.render_widget(loading_paragraph, inner_area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Page indicator
            Constraint::Min(0),    // Model list
            Constraint::Length(3), // Instructions
        ])
        .split(inner_area);

    // Calculate pagination
    let total_pages = params.models.len().div_ceil(params.models_per_page);
    let start_index = params.current_page * params.models_per_page;
    let end_index = std::cmp::min(start_index + params.models_per_page, params.models.len());

    // Page indicator
    let page_info = if total_pages > 1 {
        format!(
            "Page {} of {} ({} models)",
            params.current_page + 1,
            total_pages,
            params.models.len()
        )
    } else {
        format!("{} models", params.models.len())
    };
    let page_paragraph = Paragraph::new(page_info)
        .alignment(Alignment::Center)
        .style(params.theme.ratatui_style(Element::Inactive));
    frame.render_widget(page_paragraph, chunks[0]);

    // Get models for current page
    let page_models = &params.models[start_index..end_index];

    // Create list items for current page
    let items: Vec<ListItem> = page_models
        .iter()
        .enumerate()
        .map(|(i, (name, info))| {
            let global_index = start_index + i;
            let style = if global_index == params.selected_index {
                params.theme.highlight_style()
            } else {
                params.theme.text_style()
            };

            let line = if info.is_empty() {
                // Just show the name when there's no secondary info
                Line::from(Span::styled(
                    name.clone(),
                    style.add_modifier(Modifier::BOLD),
                ))
            } else {
                // Show name and info in columns when both are present
                Line::from(vec![
                    Span::styled(format!("{:<40}", name), style.add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:>15}", info), style),
                ])
            };

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).style(params.theme.ratatui_style(Element::Text));

    frame.render_widget(list, chunks[1]);

    // Instructions
    let instructions = if total_pages > 1 {
        "[↑↓] Navigate | [←→] Page | [ENTER] Select | [ESC] Cancel"
    } else {
        "[↑↓] Navigate | [ENTER] Select | [ESC] Cancel"
    };
    let instructions_paragraph = Paragraph::new(instructions)
        .alignment(Alignment::Center)
        .style(params.theme.ratatui_style(Element::Inactive));

    frame.render_widget(instructions_paragraph, chunks[2]);
}
