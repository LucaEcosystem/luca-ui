//! Dependency-free SVG visual backend.
//!
//! `SvgBackend` emits a static, deterministic SVG document from a
//! [`DisplayList`]. It is a *snapshot renderer*:
//! no interaction, animation, fonts, images, clipping, or accessibility
//! layer. Text is placed with the approximate cell metrics from
//! [`crate::layout_engine::text_metrics`], not with font shaping.
//!
//! The document begins with `<svg` and carries `xmlns`, `width`, `height`,
//! and `viewBox`, so it can be written directly to a `.svg` file or inlined
//! into HTML.

use std::fmt::Write as _;

use crate::geometry::Size;
use crate::layout_engine::LINE_HEIGHT_RATIO;
use crate::paint::{DisplayList, PaintCommand};
use crate::pipeline::{build_visual, Visual, DEFAULT_HEIGHT, DEFAULT_WIDTH};
use crate::rendering::{Renderer, VisualBackend};
use crate::{LucaError, ResolvedComponent};

/// Static SVG backend.
#[derive(Debug, Default)]
pub struct SvgBackend {
    output: String,
}

impl SvgBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render `root` at the default document size.
    pub fn render_to_string(root: &ResolvedComponent) -> Result<String, LucaError> {
        Self::render_to_string_with_size(root, Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
    }

    /// Render `root` at an explicit document size.
    pub fn render_to_string_with_size(
        root: &ResolvedComponent,
        size: Size,
    ) -> Result<String, LucaError> {
        let mut backend = Self::new();
        backend.render_display_list(&build_visual(root, size)?.display_list, size)?;
        Ok(backend.output)
    }

    /// Render a [`Visual`] produced by the pipeline.
    pub fn render_visual(&mut self, visual: &Visual) -> Result<(), LucaError> {
        self.render_display_list(&visual.display_list, visual.size())
    }

    /// The SVG document produced by the last render.
    pub fn output(&self) -> &str {
        &self.output
    }
}

impl VisualBackend for SvgBackend {
    fn name(&self) -> &'static str {
        "svg"
    }

    fn render_display_list(
        &mut self,
        display_list: &DisplayList,
        size: Size,
    ) -> Result<(), LucaError> {
        self.output.clear();
        write!(
            self.output,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
            size.width, size.height, size.width, size.height
        )
        .expect("writing to String cannot fail");
        for command in &display_list.commands {
            self.output.push('\n');
            match command {
                PaintCommand::Rect {
                    rect,
                    fill,
                    stroke,
                    radius,
                } => {
                    write!(
                        self.output,
                        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"",
                        rect.x, rect.y, rect.width, rect.height
                    )
                    .expect("writing to String cannot fail");
                    match fill {
                        Some(color) => {
                            write!(self.output, " fill=\"{}\"", color.to_hex())
                                .expect("writing to String cannot fail");
                        }
                        None => self.output.push_str(" fill=\"none\""),
                    }
                    if let Some(color) = stroke {
                        write!(self.output, " stroke=\"{}\"", color.to_hex())
                            .expect("writing to String cannot fail");
                    }
                    if *radius > 0 {
                        write!(self.output, " rx=\"{radius}\"")
                            .expect("writing to String cannot fail");
                    }
                    self.output.push_str("/>");
                }
                PaintCommand::Text {
                    rect,
                    text,
                    color,
                    font_size,
                } => {
                    let lines: Vec<&str> = text.split('\n').collect();
                    let baseline = rect
                        .y
                        .saturating_add((*font_size).min(i32::MAX as u32) as i32);
                    write!(
                        self.output,
                        "<text x=\"{}\" y=\"{}\" font-size=\"{}\"",
                        rect.x, baseline, font_size
                    )
                    .expect("writing to String cannot fail");
                    if let Some(color) = color {
                        write!(self.output, " fill=\"{}\"", color.to_hex())
                            .expect("writing to String cannot fail");
                    }
                    if lines.len() == 1 {
                        let escaped = escape_text(lines[0]);
                        write!(self.output, ">{escaped}</text>")
                            .expect("writing to String cannot fail");
                    } else {
                        let line_height =
                            ((*font_size as f64 * LINE_HEIGHT_RATIO).ceil() as u32).max(1);
                        self.output.push('>');
                        for (index, line) in lines.iter().enumerate() {
                            let escaped = escape_text(line);
                            if index == 0 {
                                write!(self.output, "<tspan x=\"{}\">{escaped}</tspan>", rect.x)
                                    .expect("writing to String cannot fail");
                            } else {
                                write!(
                                    self.output,
                                    "<tspan x=\"{}\" dy=\"{}\">{escaped}</tspan>",
                                    rect.x, line_height
                                )
                                .expect("writing to String cannot fail");
                            }
                        }
                        self.output.push_str("</text>");
                    }
                }
            }
        }
        self.output.push_str("\n</svg>");
        Ok(())
    }
}

impl Renderer for SvgBackend {
    fn name(&self) -> &'static str {
        "svg"
    }

    fn render(&mut self, root: &ResolvedComponent) -> Result<(), LucaError> {
        let visual = build_visual(root, Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))?;
        self.render_visual(&visual)
    }
}

fn escape_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(c),
        }
    }
    escaped
}
