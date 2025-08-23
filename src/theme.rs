//! Theme system for Agentic
//!
//! Implements the "Karesansui" zen garden aesthetic with minimalist design principles.

use ratatui::style::{Color, Style};

/// Color palette inspired by zen garden aesthetics
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub text: Color,
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Yellow,
            background: Color::Black,
            text: Color::White,
            border: Color::Gray,
        }
    }
}

impl Theme {
    /// Create a new theme instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Get primary style
    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    /// Get secondary style
    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    /// Get accent style
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Get text style
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text)
    }

    /// Get border style
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }
}
