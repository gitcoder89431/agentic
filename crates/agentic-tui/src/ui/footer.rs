use crate::ui::app::AppMode;
use agentic_core::theme::{Element, Theme};
use ratatui::{
    prelude::{Alignment, Frame, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render_footer(frame: &mut Frame, area: Rect, theme: &Theme, mode: AppMode, chat_input: &str) {
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .style(theme.ratatui_style(Element::Active));

    let inner_area = footer_block.inner(area);
    
    let content = match mode {
        AppMode::Chat => {
            // Chat input field with cursor
            let display_text = if chat_input.is_empty() {
                "Share your query with Ruixen... (/settings, /quit)"
            } else {
                chat_input
            };
            
            let mut spans = vec![
                Span::styled("💬 ", theme.ratatui_style(Element::Accent)),
                Span::styled(display_text, theme.text_style()),
            ];
            
            // Add cursor when in chat mode
            if !chat_input.is_empty() || area.width > 50 {
                spans.push(Span::styled("_", theme.highlight_style()));
            }
            
            Line::from(spans)
        }
        _ => {
            // Navigation bar for Normal mode
            Line::from(vec![
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
            .alignment(Alignment::Center)
        }
    };

    let footer_paragraph = Paragraph::new(content).style(theme.ratatui_style(Element::Text));

    frame.render_widget(footer_block, area);
    frame.render_widget(footer_paragraph, inner_area);
}
