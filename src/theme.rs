//! Everforest Theme System for Agentic
//!
//! Implements a comprehensive theming architecture with Everforest Dark/Light variants.
//! Provides clean separation of concerns and runtime theme switching capability.

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Theme variants supported by Agentic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeVariant {
    /// Everforest Dark theme (default)
    EverforestDark,
    /// Everforest Light theme
    EverforestLight,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        Self::EverforestDark
    }
}

/// Color palette for a theme variant
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub secondary: Color,
    pub info: Color,
    pub border: Color,
    pub selection: Color,
    pub cursor: Color,
    pub warning: Color,  // Yellow/orange for settings
}

/// UI element types for styling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    /// Normal text content
    Text,
    /// Titles and headers
    Title,
    /// Borders and frames
    Border,
    /// Highlighted/selected items
    Highlight,
    /// Accent elements (buttons, links)
    Accent,
    /// Secondary elements
    Secondary,
    /// Information/status elements
    Info,
    /// Background elements
    Background,
    /// Active/focused elements
    Active,
    /// Inactive/disabled elements
    Inactive,
    /// Warning/settings elements (yellow/orange)
    Warning,
}

/// Main theme structure managing all UI styling
#[derive(Debug, Clone)]
pub struct Theme {
    variant: ThemeVariant,
    colors: ColorPalette,
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(ThemeVariant::default())
    }
}

impl Theme {
    /// Create a new theme with the specified variant
    pub fn new(variant: ThemeVariant) -> Self {
        let colors = match variant {
            ThemeVariant::EverforestDark => ColorPalette {
                background: Color::Rgb(45, 53, 59),    // #2d353b
                foreground: Color::Rgb(211, 198, 170), // #d3c6aa
                accent: Color::Rgb(167, 192, 128),     // #a7c080 (green)
                secondary: Color::Rgb(230, 126, 128),  // #e67e80 (red)
                info: Color::Rgb(127, 187, 179),       // #7fbbb3 (aqua)
                border: Color::Rgb(116, 125, 135),     // #747d87 (gray)
                selection: Color::Rgb(64, 72, 78),     // #40484e (darker bg)
                cursor: Color::Rgb(211, 198, 170),     // #d3c6aa (same as fg)
                warning: Color::Rgb(219, 188, 127),    // #dbbc7f (yellow/orange)
            },
            ThemeVariant::EverforestLight => ColorPalette {
                background: Color::Rgb(253, 246, 227), // #fdf6e3
                foreground: Color::Rgb(92, 106, 114),  // #5c6a72
                accent: Color::Rgb(141, 161, 1),       // #8da101 (green)
                secondary: Color::Rgb(248, 85, 82),    // #f85552 (red)
                info: Color::Rgb(53, 167, 124),        // #35a77c (aqua)
                border: Color::Rgb(150, 160, 170),     // #96a0aa (gray)
                selection: Color::Rgb(243, 236, 217),  // #f3ecd9 (darker bg)
                cursor: Color::Rgb(92, 106, 114),      // #5c6a72 (same as fg)
                warning: Color::Rgb(207, 131, 44),     // #cf832c (yellow/orange)
            },
        };

        Self { variant, colors }
    }

    /// Get the current theme variant
    pub fn variant(&self) -> ThemeVariant {
        self.variant
    }

    /// Get the color palette
    pub fn colors(&self) -> &ColorPalette {
        &self.colors
    }

    /// Toggle between dark and light variants
    pub fn toggle(&mut self) {
        self.variant = match self.variant {
            ThemeVariant::EverforestDark => ThemeVariant::EverforestLight,
            ThemeVariant::EverforestLight => ThemeVariant::EverforestDark,
        };
        *self = Self::new(self.variant);
    }

    /// Set specific theme variant
    pub fn set_variant(&mut self, variant: ThemeVariant) {
        if self.variant != variant {
            self.variant = variant;
            *self = Self::new(self.variant);
        }
    }

    /// Get a ratatui Style for the specified UI element
    pub fn ratatui_style(&self, element: Element) -> Style {
        match element {
            Element::Text => Style::default()
                .fg(self.colors.foreground)
                .bg(self.colors.background),

            Element::Title => Style::default()
                .fg(self.colors.accent)
                .bg(self.colors.background)
                .add_modifier(Modifier::BOLD),

            Element::Border => Style::default()
                .fg(self.colors.border)
                .bg(self.colors.background),

            Element::Highlight => Style::default()
                .fg(self.colors.foreground)
                .bg(self.colors.selection)
                .add_modifier(Modifier::BOLD),

            Element::Accent => Style::default()
                .fg(self.colors.accent)
                .bg(self.colors.background)
                .add_modifier(Modifier::BOLD),

            Element::Secondary => Style::default()
                .fg(self.colors.secondary)
                .bg(self.colors.background),

            Element::Info => Style::default()
                .fg(self.colors.info)
                .bg(self.colors.background),

            Element::Background => Style::default()
                .fg(self.colors.foreground)
                .bg(self.colors.background),

            Element::Active => Style::default()
                .fg(self.colors.accent)
                .bg(self.colors.selection)
                .add_modifier(Modifier::BOLD),

            Element::Inactive => Style::default()
                .fg(self.colors.border)
                .bg(self.colors.background),

            Element::Warning => Style::default()
                .fg(self.colors.warning)
                .bg(self.colors.background),
        }
    }

    /// Get foreground color for an element
    pub fn fg_color(&self, element: Element) -> Color {
        match element {
            Element::Text | Element::Background => self.colors.foreground,
            Element::Title | Element::Accent | Element::Active => self.colors.accent,
            Element::Border | Element::Inactive => self.colors.border,
            Element::Highlight => self.colors.foreground,
            Element::Secondary => self.colors.secondary,
            Element::Info => self.colors.info,
            Element::Warning => self.colors.warning,
        }
    }

    /// Get background color for an element
    pub fn bg_color(&self, element: Element) -> Color {
        match element {
            Element::Highlight | Element::Active => self.colors.selection,
            _ => self.colors.background,
        }
    }

    /// Get style for block titles
    pub fn title_style(&self) -> Style {
        self.ratatui_style(Element::Title)
    }

    /// Get style for block borders
    pub fn border_style(&self) -> Style {
        self.ratatui_style(Element::Border)
    }

    /// Get style for normal text
    pub fn text_style(&self) -> Style {
        self.ratatui_style(Element::Text)
    }

    /// Get style for highlighted/selected items
    pub fn highlight_style(&self) -> Style {
        self.ratatui_style(Element::Highlight)
    }

    /// Get style for accent elements
    pub fn accent_style(&self) -> Style {
        self.ratatui_style(Element::Accent)
    }

    /// Get style for secondary elements
    pub fn secondary_style(&self) -> Style {
        self.ratatui_style(Element::Secondary)
    }

    /// Get style for info elements
    pub fn info_style(&self) -> Style {
        self.ratatui_style(Element::Info)
    }

    /// Get style for warning/settings elements
    pub fn warning_style(&self) -> Style {
        self.ratatui_style(Element::Warning)
    }
}
