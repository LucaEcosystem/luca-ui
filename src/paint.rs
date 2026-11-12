//! Display-list painting for the static visual backend.
//!
//! A display list is a flat, backend-independent sequence of primitive
//! commands in paint order. It is produced from a [`LayoutNode`] tree:
//! parents before children, and within each node the background/border
//! rectangle first, then its text, then its children. There is no clipping,
//! compositing, opacity, or animation in this milestone.

use crate::geometry::{Rect, Size};
use crate::layout_engine::LayoutNode;
use crate::style::Color;

/// One primitive paint command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaintCommand {
    /// A filled and/or stroked rectangle. At least one of `fill`/`stroke`
    /// is present; `radius` is a uniform corner radius.
    Rect {
        rect: Rect,
        fill: Option<Color>,
        stroke: Option<Color>,
        radius: u32,
    },
    /// One text run positioned at the top-left of `rect`.
    Text {
        rect: Rect,
        text: String,
        color: Option<Color>,
        font_size: u32,
    },
}

/// A flat paint sequence plus the document size it was built for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DisplayList {
    pub size: Size,
    pub commands: Vec<PaintCommand>,
}

impl DisplayList {
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }
}

/// Build the display list for a laid-out tree.
pub fn build_display_list(root: &LayoutNode) -> DisplayList {
    let mut display_list = DisplayList {
        size: root.rect.size(),
        commands: Vec::new(),
    };
    paint(root, &mut display_list.commands);
    display_list
}

fn paint(node: &LayoutNode, commands: &mut Vec<PaintCommand>) {
    if node.style.background.is_some() || node.style.border.is_some() {
        commands.push(PaintCommand::Rect {
            rect: node.rect,
            fill: node.style.background,
            stroke: node.style.border,
            radius: node.style.radius,
        });
    }
    if let Some(text) = &node.text {
        commands.push(PaintCommand::Text {
            rect: node.content_rect(),
            text: text.clone(),
            color: node.style.color,
            font_size: node.style.font_size,
        });
    }
    for child in &node.children {
        paint(child, commands);
    }
}
