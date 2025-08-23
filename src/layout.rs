//! Taffy Layout System for Agentic
//!
//! Implements a production-grade 3-layer layout (Header, Body, Footer) using Taffy's
//! flexbox-style layout engine for responsive terminal layouts.

use taffy::{
    style::{Dimension, FlexDirection, Style},
    TaffyTree, NodeId,
};
use ratatui::layout::Rect;

/// Layout rectangles for the 3-layer application structure
#[derive(Debug, Clone)]
pub struct LayoutRects {
    /// Header section rectangle
    pub header: Rect,
    /// Main body content rectangle  
    pub body: Rect,
    /// Footer section rectangle
    pub footer: Rect,
}

/// Main application layout manager using Taffy flexbox engine
pub struct AppLayout {
    /// Taffy layout engine instance
    taffy: TaffyTree,
    /// Root container node
    root: NodeId,
    /// Header section node
    header: NodeId,
    /// Body section node
    body: NodeId,
    /// Footer section node
    footer: NodeId,
}

impl AppLayout {
    /// Create a new 3-layer layout with Taffy
    pub fn new() -> Result<Self, taffy::TaffyError> {
        let mut taffy = TaffyTree::new();

        // Create header node with fixed height (3 rows)
        let header = taffy.new_leaf(Style {
            size: taffy::geometry::Size {
                width: Dimension::Percent(1.0), // 100% width
                height: Dimension::Length(3.0), // Fixed 3 rows height
            },
            ..Default::default()
        })?;

        // Create body node with flex-grow to fill remaining space
        let body = taffy.new_leaf(Style {
            size: taffy::geometry::Size {
                width: Dimension::Percent(1.0), // 100% width
                height: Dimension::Auto,        // Auto height, will flex-grow
            },
            flex_grow: 1.0, // Grow to fill available space
            ..Default::default()
        })?;

        // Create footer node with fixed height (3 rows)
        let footer = taffy.new_leaf(Style {
            size: taffy::geometry::Size {
                width: Dimension::Percent(1.0), // 100% width
                height: Dimension::Length(3.0), // Fixed 3 rows height
            },
            ..Default::default()
        })?;

        // Create root container with vertical flex direction
        let root = taffy.new_with_children(Style {
            size: taffy::geometry::Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Percent(1.0),
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        }, &[header, body, footer])?;

        Ok(Self {
            taffy,
            root,
            header,
            body,
            footer,
        })
    }

    /// Compute layout for the given terminal size and return layout rectangles
    pub fn compute(&mut self, terminal_size: (u16, u16)) -> Result<LayoutRects, taffy::TaffyError> {
        let (width, height) = terminal_size;

        // Set the available space for layout computation
        let available_space = taffy::geometry::Size {
            width: taffy::AvailableSpace::Definite(width as f32),
            height: taffy::AvailableSpace::Definite(height as f32),
        };

        // Compute the layout
        self.taffy.compute_layout(self.root, available_space)?;

        // Get computed layouts for each section
        let _root_layout = self.taffy.layout(self.root)?;
        let header_layout = self.taffy.layout(self.header)?;
        let body_layout = self.taffy.layout(self.body)?;
        let footer_layout = self.taffy.layout(self.footer)?;

        // Convert Taffy layouts to ratatui Rects
        let header = Rect {
            x: header_layout.location.x as u16,
            y: header_layout.location.y as u16,
            width: header_layout.size.width as u16,
            height: header_layout.size.height as u16,
        };

        let body = Rect {
            x: body_layout.location.x as u16,
            y: body_layout.location.y as u16,
            width: body_layout.size.width as u16,
            height: body_layout.size.height as u16,
        };

        let footer = Rect {
            x: footer_layout.location.x as u16,
            y: footer_layout.location.y as u16,
            width: footer_layout.size.width as u16,
            height: footer_layout.size.height as u16,
        };

        Ok(LayoutRects {
            header,
            body,
            footer,
        })
    }

    /// Get the current layout rectangles (header, body, footer)
    /// 
    /// Note: This requires compute() to be called first with current terminal size
    pub fn get_rects(&self) -> Result<(Rect, Rect, Rect), taffy::TaffyError> {
        let header_layout = self.taffy.layout(self.header)?;
        let body_layout = self.taffy.layout(self.body)?;
        let footer_layout = self.taffy.layout(self.footer)?;

        let header = Rect {
            x: header_layout.location.x as u16,
            y: header_layout.location.y as u16,
            width: header_layout.size.width as u16,
            height: header_layout.size.height as u16,
        };

        let body = Rect {
            x: body_layout.location.x as u16,
            y: body_layout.location.y as u16,
            width: body_layout.size.width as u16,
            height: body_layout.size.height as u16,
        };

        let footer = Rect {
            x: footer_layout.location.x as u16,
            y: footer_layout.location.y as u16,
            width: footer_layout.size.width as u16,
            height: footer_layout.size.height as u16,
        };

        Ok((header, body, footer))
    }
}

impl Default for AppLayout {
    fn default() -> Self {
        Self::new().expect("Failed to create default AppLayout")
    }
}
