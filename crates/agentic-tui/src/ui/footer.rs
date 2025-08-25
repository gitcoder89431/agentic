use agentic_core::theme::{Element, Theme};
use ratatui::{
    prelude::{Alignment, Frame, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render_footer(frame: &mut Frame, area: Rect, theme: &Theme) {
    let nav_text = Line::from(vec![
        Span::raw("[A]"),
        Span::styled("bout", theme.ratatui_style(Element::Inactive)),
        Span::raw(" | "),
        Span::raw("[S]"),
        Span::styled("ettings", theme.ratatui_style(Element::Inactive)),
        Span::raw(" | "),
        Span::raw("[T]"),
        Span::styled("heme", theme.ratatui_style(Element::Inactive)),
        Span::raw(" | "),
        Span::raw("[Q]"),
        Span::styled("uit", theme.ratatui_style(Element::Inactive)),
    ])
    .alignment(Alignment::Center);

    let footer_block = Block::default()
        .borders(Borders::ALL)
        .style(theme.ratatui_style(Element::Active));

    let inner_area = footer_block.inner(area);

    let footer_paragraph = Paragraph::new(nav_text).style(theme.ratatui_style(Element::Text));

    frame.render_widget(footer_block, area);
    frame.render_widget(footer_paragraph, inner_area);
}
