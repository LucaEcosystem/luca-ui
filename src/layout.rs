//! Platform-independent layout representation and the documented ◐
//! assumption subset.
//!
//! Locked design facts: layout is part of the declarative
//! layer; the UI API is platform-independent and backed by a rendering
//! abstraction; no platform-specific components in the core vocabulary; a
//! development inspector may show layout bounds.
//!
//! `Layout` is a validated, backend-agnostic container of generic
//! `key = scalar` entries attached to one component. Structural validation
//! only: identifier keys, scalar values, no duplicates, no imperative
//! operations, source-located Luca errors.
//!
//! [`LayoutSpec`] interprets the ◐ assumption subset (`direction`, `gap`,
//! `padding`, `width`, `height`, `align`) for the visual pipeline. Unknown
//! entries stay opaque: they are accepted, preserved, and ignored by the
//! visual layer. Invalid values for recognized entries produce
//! [`RenderDiagnostic`]s rather than parse errors.

use crate::bindings::ResolvedProp;
use crate::rendering::RenderDiagnostic;
use crate::{LucaError, Prop};

/// Main-axis stacking direction for the documented ◐ layout assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Children stack top to bottom.
    #[default]
    Column,
    /// Children stack left to right.
    Row,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Column => "column",
            Direction::Row => "row",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "column" => Some(Direction::Column),
            "row" => Some(Direction::Row),
            _ => None,
        }
    }
}

/// Cross-axis alignment for the documented ◐ layout assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    /// Fill the parent's cross-axis content size (when the child has no
    /// explicit cross-axis size).
    Stretch,
}

impl Align {
    pub fn as_str(self) -> &'static str {
        match self {
            Align::Start => "start",
            Align::Center => "center",
            Align::End => "end",
            Align::Stretch => "stretch",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "start" => Some(Align::Start),
            "center" => Some(Align::Center),
            "end" => Some(Align::End),
            "stretch" => Some(Align::Stretch),
            _ => None,
        }
    }
}

/// The documented ◐ layout assumption interpreted from a `layout:` block.
///
/// Recognized entries are `direction`, `gap`, `padding`, `width`, `height`,
/// and `align`. Unknown entries stay opaque (they are silently ignored by the
/// visual layer, never rejected). Invalid values for recognized entries
/// produce [`RenderDiagnostic`]s and fall back to the default. Sizes are
/// non-negative integers in abstract units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutSpec {
    pub direction: Direction,
    /// Space between siblings along the main axis.
    pub gap: u32,
    /// Uniform inset between a node's border box and its content.
    pub padding: u32,
    /// Explicit width; `None` means intrinsic size (the root defaults to the
    /// viewport instead).
    pub width: Option<u32>,
    /// Explicit height; `None` means intrinsic size (the root defaults to the
    /// viewport instead).
    pub height: Option<u32>,
    pub align: Align,
}

impl Default for LayoutSpec {
    fn default() -> Self {
        Self {
            direction: Direction::Column,
            gap: 0,
            padding: 0,
            width: None,
            height: None,
            align: Align::Start,
        }
    }
}

impl LayoutSpec {
    /// Interpret resolved `layout:` entries under the documented assumption.
    pub fn from_entries(
        component: &str,
        entries: &[ResolvedProp],
    ) -> (Self, Vec<RenderDiagnostic>) {
        let mut spec = Self::default();
        let mut diagnostics = Vec::new();
        for entry in entries {
            match entry.key.as_str() {
                "direction" => match &entry.value {
                    luca_code::types::Value::Str(text) => match Direction::parse(text) {
                        Some(direction) => spec.direction = direction,
                        None => diagnostics.push(
                            RenderDiagnostic::new(
                                format!(
                                    "Unknown layout direction {text:?}; expected \"column\" or \"row\""
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
                                "Layout entry 'direction' must be str, found {}",
                                value_kind(other)
                            ),
                            entry.line,
                            entry.column,
                        )
                        .on_component(component),
                    ),
                },
                "align" => match &entry.value {
                    luca_code::types::Value::Str(text) => match Align::parse(text) {
                        Some(align) => spec.align = align,
                        None => diagnostics.push(
                            RenderDiagnostic::new(
                                format!(
                                    "Unknown layout align {text:?}; expected \"start\", \"center\", \"end\", or \"stretch\""
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
                                "Layout entry 'align' must be str, found {}",
                                value_kind(other)
                            ),
                            entry.line,
                            entry.column,
                        )
                        .on_component(component),
                    ),
                },
                "gap" | "padding" | "width" | "height" => {
                    let slot = match entry.key.as_str() {
                        "gap" => &mut spec.gap,
                        "padding" => &mut spec.padding,
                        "width" => {
                            if let Some(value) = non_negative(component, entry, &mut diagnostics) {
                                spec.width = Some(value);
                            }
                            continue;
                        }
                        "height" => {
                            if let Some(value) = non_negative(component, entry, &mut diagnostics) {
                                spec.height = Some(value);
                            }
                            continue;
                        }
                        _ => unreachable!(),
                    };
                    if let Some(value) = non_negative(component, entry, &mut diagnostics) {
                        *slot = value;
                    }
                }
                // Unknown entries stay opaque by design.
                _ => {}
            }
        }
        (spec, diagnostics)
    }
}

fn non_negative(
    component: &str,
    entry: &ResolvedProp,
    diagnostics: &mut Vec<RenderDiagnostic>,
) -> Option<u32> {
    match &entry.value {
        luca_code::types::Value::Int(value) if *value >= 0 && *value <= u32::MAX as i64 => {
            Some(*value as u32)
        }
        other => {
            diagnostics.push(
                RenderDiagnostic::new(
                    format!(
                        "Layout entry '{}' must be a non-negative integer, found {}",
                        entry.key,
                        value_kind(other)
                    ),
                    entry.line,
                    entry.column,
                )
                .on_component(component),
            );
            None
        }
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

/// Validated layout entries belonging to one component.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub entries: Vec<Prop>,
    pub line: usize,
    pub column: usize,
}

impl Layout {
    /// Validate raw entries harvested from a `layout:` block.
    pub(crate) fn from_entries(
        component: &str,
        entries: Vec<Prop>,
        line: usize,
        column: usize,
    ) -> Result<Self, LucaError> {
        check_block_entries("layout", component, &entries)?;
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

/// Shared structural checks for `layout:`/`style:` entry lists.
pub(crate) fn check_block_entries(
    block: &str,
    component: &str,
    entries: &[Prop],
) -> Result<(), LucaError> {
    use std::collections::HashMap;
    let mut seen: HashMap<&str, (usize, usize)> = HashMap::new();
    for entry in entries {
        if let Some((line, column)) = seen.get(entry.key.as_str()) {
            return Err(LucaError::new(
                format!(
                    "Duplicate '{key}' in `{block}` block of component '{component}' (first at {line}:{column})",
                    key = entry.key,
                ),
                entry.line,
                entry.column,
            ));
        }
        seen.insert(entry.key.as_str(), (entry.line, entry.column));
        if ["hide", "show", "set_text"].contains(&entry.key.as_str()) {
            return Err(LucaError::new(
                format!(
                    "Imperative component operation '{}' is not supported; update state and allow the UI to react",
                    entry.key
                ),
                entry.line,
                entry.column,
            ));
        }
    }
    Ok(())
}
