//! Platform-independent styling representation and the documented ◐
//! assumption subset.
//!
//! Locked design facts: styling is part of the declarative
//! layer; the UI API is platform-independent and backed by a rendering
//! abstraction; no platform-specific components in the core vocabulary.
//!
//! `Style` is a validated, backend-agnostic container of generic
//! `key = scalar` entries attached to one component. Structural validation
//! only: identifier keys, scalar values, no duplicates, no imperative
//! operations, source-located Luca errors.
//!
//! [`StyleSpec`] interprets the ◐ assumption subset (`background`, `color`,
//! `border`, `radius`, `font_size`) for the visual pipeline. Colors are
//! `#rrggbb`; unknown tokens stay opaque. Invalid values for recognized
//! tokens produce [`RenderDiagnostic`]s rather than parse errors.

use crate::bindings::ResolvedProp;
use crate::layout::check_block_entries;
use crate::rendering::RenderDiagnostic;
use crate::{LucaError, Prop};

/// An sRGB color parsed from a `#rrggbb` token. Only six-digit hex is
/// accepted; named colors and other formats stay opaque to the visual layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Parse `#rrggbb` (case-insensitive). Returns `None` for anything else.
    pub fn parse(text: &str) -> Option<Self> {
        let hex = text.strip_prefix('#')?;
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    /// Deterministic lowercase `#rrggbb` form.
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// The documented ◐ style assumption interpreted from a `style:` block.
///
/// Recognized tokens are `background`, `color`, `border`, `radius`, and
/// `font_size`; colors use `#rrggbb`. Unknown tokens stay opaque. Invalid
/// values for recognized tokens produce [`RenderDiagnostic`]s and fall back
/// to the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleSpec {
    pub background: Option<Color>,
    pub color: Option<Color>,
    pub border: Option<Color>,
    pub radius: u32,
    pub font_size: u32,
}

impl Default for StyleSpec {
    fn default() -> Self {
        Self {
            background: None,
            color: None,
            border: None,
            radius: 0,
            font_size: 16,
        }
    }
}

impl StyleSpec {
    /// Interpret resolved `style:` entries under the documented assumption.
    pub fn from_entries(
        component: &str,
        entries: &[ResolvedProp],
    ) -> (Self, Vec<RenderDiagnostic>) {
        let mut spec = Self::default();
        let mut diagnostics = Vec::new();
        for entry in entries {
            match entry.key.as_str() {
                "background" | "color" | "border" => {
                    let slot = match entry.key.as_str() {
                        "background" => &mut spec.background,
                        "color" => &mut spec.color,
                        "border" => &mut spec.border,
                        _ => unreachable!(),
                    };
                    match &entry.value {
                        luca_code::types::Value::Str(text) => match Color::parse(text) {
                            Some(color) => *slot = Some(color),
                            None => diagnostics.push(
                                RenderDiagnostic::new(
                                    format!(
                                        "Style entry '{}' must be a #rrggbb color, found {text:?}",
                                        entry.key
                                    ),
                                    entry.line,
                                    entry.column,
                                )
                                .on_component(component),
                            ),
                        },
                        other => diagnostics.push(
                            RenderDiagnostic::new(
                                format!(
                                    "Style entry '{}' must be str, found {}",
                                    entry.key,
                                    value_kind(other)
                                ),
                                entry.line,
                                entry.column,
                            )
                            .on_component(component),
                        ),
                    }
                }
                "radius" | "font_size" => {
                    let slot = if entry.key == "radius" {
                        &mut spec.radius
                    } else {
                        &mut spec.font_size
                    };
                    match &entry.value {
                        luca_code::types::Value::Int(value)
                            if *value >= 0 && *value <= u32::MAX as i64 =>
                        {
                            *slot = *value as u32
                        }
                        other => diagnostics.push(
                            RenderDiagnostic::new(
                                format!(
                                    "Style entry '{}' must be a non-negative integer, found {}",
                                    entry.key,
                                    value_kind(other)
                                ),
                                entry.line,
                                entry.column,
                            )
                            .on_component(component),
                        ),
                    }
                }
                // Unknown tokens stay opaque by design.
                _ => {}
            }
        }
        (spec, diagnostics)
    }
}

fn value_kind(value: &luca_code::types::Value) -> &'static str {
    match value {
        luca_code::types::Value::Int(_) => "int",
        luca_code::types::Value::Dec(_) => "dec",
        luca_code::types::Value::Str(_) => "str",
        luca_code::types::Value::Bool(_) => "bool",
        luca_code::types::Value::None => "none",
        _ => "collection",
    }
}

/// Validated style entries belonging to one component.
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub entries: Vec<Prop>,
    pub line: usize,
    pub column: usize,
}

impl Style {
    /// Validate raw entries harvested from a `style:` block.
    pub(crate) fn from_entries(
        component: &str,
        entries: Vec<Prop>,
        line: usize,
        column: usize,
    ) -> Result<Self, LucaError> {
        check_block_entries("style", component, &entries)?;
        Ok(Self {
            entries,
            line,
            column,
        })
    }

    /// True when the block declares no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
