//! Geometry engine for the documented ◐ layout assumption.
//!
//! This is deliberately small and deterministic: column/row stacking,
//! uniform padding, a main-axis gap, cross-axis alignment, and optional
//! explicit sizes. There is no wrapping, clipping, scrolling, flex basis,
//! grid, or text shaping. Text is measured with a documented approximate
//! cell model (see [`text_metrics`]), not with font metrics.
//!
//! The root fills the viewport unless it declares `width`/`height`.
//! Containers size to their content unless they declare a size. Hidden
//! subtrees are excluded before measurement, so they neither occupy space
//! nor receive paint commands.

use crate::bindings::ResolvedComponent;
use crate::geometry::{Rect, Size};
use crate::layout::{Align, Direction, LayoutSpec};
use crate::rendering::RenderDiagnostic;
use crate::style::StyleSpec;
use crate::LucaError;

/// Approximate advance width of one character as a fraction of font size.
/// Part of the ◐ text-metrics assumption; no shaping or kerning is applied.
pub const CELL_WIDTH_RATIO: f64 = 0.6;

/// Approximate line height as a fraction of font size. ◐ assumption.
pub const LINE_HEIGHT_RATIO: f64 = 1.2;

/// Measure one text block with the documented approximate cell model:
/// character count × `font_size` × 0.6 wide, line count × `font_size` × 1.2
/// tall. Strings are split on `\n`; Unicode scalar values each advance one
/// cell. This is deterministic and platform-independent, not typographic.
pub fn text_metrics(text: &str, font_size: u32) -> Size {
    let font_size = font_size.max(1);
    let cell = (font_size as f64 * CELL_WIDTH_RATIO).ceil() as u32;
    let line_height = (font_size as f64 * LINE_HEIGHT_RATIO).ceil() as u32;
    let mut width = 0u32;
    let mut lines = 0u32;
    for line in text.split('\n') {
        lines = lines.saturating_add(1);
        width = width.max((line.chars().count() as u32).saturating_mul(cell));
    }
    Size::new(width, lines.saturating_mul(line_height))
}

/// One node in the computed layout tree. Paint order and hit testing both
/// follow this tree's depth-first, parent-first order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutNode {
    pub name: String,
    pub id: Option<String>,
    /// Outer (border-box) rectangle in abstract units.
    pub rect: Rect,
    pub layout: LayoutSpec,
    pub style: StyleSpec,
    /// Text drawn for this node, from the generic `text` property.
    pub text: Option<String>,
    pub children: Vec<LayoutNode>,
    pub line: usize,
    pub column: usize,
}

impl LayoutNode {
    /// The content rectangle inside `padding`.
    pub fn content_rect(&self) -> Rect {
        self.rect.inset(self.layout.padding)
    }

    /// Find a laid-out node by component `id` (depth-first).
    pub fn find_by_id(&self, id: &str) -> Option<&LayoutNode> {
        if self.id.as_deref() == Some(id) {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find_by_id(id))
    }

    /// Visit this node and its descendants depth-first, parents first.
    pub fn visit(&self, f: &mut impl FnMut(&LayoutNode)) {
        f(self);
        for child in &self.children {
            child.visit(f);
        }
    }
}

struct PendingNode {
    name: String,
    id: Option<String>,
    layout: LayoutSpec,
    style: StyleSpec,
    text: Option<String>,
    children: Vec<PendingNode>,
    line: usize,
    column: usize,
}

/// Compute layout for a resolved tree inside `viewport`.
///
/// Returns the laid-out root and every non-fatal diagnostic produced by the
/// documented layout/style assumptions. Invalid known entries are reported
/// and ignored; unknown entries stay opaque.
pub fn compute_layout(
    root: &ResolvedComponent,
    viewport: Size,
) -> Result<(LayoutNode, Vec<RenderDiagnostic>), LucaError> {
    let mut diagnostics = Vec::new();
    let pending = build_pending(root, &mut diagnostics);
    let width = pending.layout.width.unwrap_or(viewport.width);
    let height = pending.layout.height.unwrap_or(viewport.height);
    let root = place(pending, Rect::new(0, 0, width, height));
    Ok((root, diagnostics))
}

fn build_pending(node: &ResolvedComponent, diagnostics: &mut Vec<RenderDiagnostic>) -> PendingNode {
    let (layout, mut layout_diagnostics) = LayoutSpec::from_entries(&node.name, &node.layout);
    diagnostics.append(&mut layout_diagnostics);
    let (style, mut style_diagnostics) = StyleSpec::from_entries(&node.name, &node.style);
    diagnostics.append(&mut style_diagnostics);
    let text = match node.prop("text") {
        Some(value) => display_text(value),
        None => None,
    };
    let children = node
        .children
        .iter()
        .filter(|child| crate::is_visible(child))
        .map(|child| build_pending(child, diagnostics))
        .collect();
    PendingNode {
        name: node.name.clone(),
        id: node.id.clone(),
        layout,
        style,
        text,
        children,
        line: node.line,
        column: node.column,
    }
}

fn display_text(value: &luca_code::types::Value) -> Option<String> {
    match value {
        luca_code::types::Value::Str(text) if text.is_empty() => None,
        luca_code::types::Value::Str(text) => Some(text.clone()),
        other => Some(other.display()),
    }
}

fn measure(node: &PendingNode) -> Size {
    let text_size = node
        .text
        .as_deref()
        .map(|text| text_metrics(text, node.style.font_size))
        .unwrap_or_default();
    let content = if node.children.is_empty() {
        text_size
    } else {
        let child_size = match node.layout.direction {
            Direction::Column => {
                let mut width = 0u32;
                let mut height = 0u32;
                for (index, child) in node.children.iter().enumerate() {
                    if index > 0 {
                        height = height.saturating_add(node.layout.gap);
                    }
                    let size = measure(child);
                    width = width.max(size.width);
                    height = height.saturating_add(size.height);
                }
                Size::new(width, height)
            }
            Direction::Row => {
                let mut width = 0u32;
                let mut height = 0u32;
                for (index, child) in node.children.iter().enumerate() {
                    if index > 0 {
                        width = width.saturating_add(node.layout.gap);
                    }
                    let size = measure(child);
                    width = width.saturating_add(size.width);
                    height = height.max(size.height);
                }
                Size::new(width, height)
            }
        };
        // A container that also carries text reserves room for both.
        Size::new(
            child_size.width.max(text_size.width),
            child_size.height.max(text_size.height),
        )
    };
    let padding = node.layout.padding.saturating_mul(2);
    Size::new(
        node.layout
            .width
            .unwrap_or_else(|| content.width.saturating_add(padding)),
        node.layout
            .height
            .unwrap_or_else(|| content.height.saturating_add(padding)),
    )
}

fn place(node: PendingNode, rect: Rect) -> LayoutNode {
    let content = rect.inset(node.layout.padding);
    let gap = node.layout.gap;
    let mut children = Vec::with_capacity(node.children.len());
    match node.layout.direction {
        Direction::Column => {
            let mut cursor = content.y;
            for child in node.children {
                let size = measure(&child);
                let width = stretch_cross(
                    node.layout.align,
                    content.width,
                    size.width,
                    child.layout.width.is_some(),
                );
                let x = cross_position(node.layout.align, content, width, Direction::Column);
                let child_rect = Rect::new(x, cursor, width, size.height);
                cursor = cursor
                    .saturating_add(size.height.min(i32::MAX as u32) as i32)
                    .saturating_add(gap.min(i32::MAX as u32) as i32);
                children.push(place(child, child_rect));
            }
        }
        Direction::Row => {
            let mut cursor = content.x;
            for child in node.children {
                let size = measure(&child);
                let height = stretch_cross(
                    node.layout.align,
                    content.height,
                    size.height,
                    child.layout.height.is_some(),
                );
                let y = cross_position(node.layout.align, content, height, Direction::Row);
                let child_rect = Rect::new(cursor, y, size.width, height);
                cursor = cursor
                    .saturating_add(size.width.min(i32::MAX as u32) as i32)
                    .saturating_add(gap.min(i32::MAX as u32) as i32);
                children.push(place(child, child_rect));
            }
        }
    }
    LayoutNode {
        name: node.name,
        id: node.id,
        rect,
        layout: node.layout,
        style: node.style,
        text: node.text,
        children,
        line: node.line,
        column: node.column,
    }
}

fn stretch_cross(align: Align, available: u32, measured: u32, explicit: bool) -> u32 {
    if align == Align::Stretch && !explicit {
        available
    } else {
        measured
    }
}

fn cross_position(align: Align, content: Rect, size: u32, direction: Direction) -> i32 {
    let available = match direction {
        Direction::Column => content.width,
        Direction::Row => content.height,
    };
    let start = match direction {
        Direction::Column => content.x,
        Direction::Row => content.y,
    };
    match align {
        Align::Start | Align::Stretch => start,
        Align::Center => start.saturating_add((available.saturating_sub(size) / 2) as i32),
        Align::End => start.saturating_add(available.saturating_sub(size) as i32),
    }
}
