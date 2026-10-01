//! Platform-independent layout representation for 0.3 Alpha.
//!
//! Locked facts from `docs/SPEC.md`: layout is part of the declarative
//! layer; the UI API is platform-independent and backed by a rendering
//! abstraction; no platform-specific components in the core vocabulary; a
//! development inspector may show layout bounds. Nothing else about layout
//! is locked: entry names, dimensions, defaults, and precedence rules must
//! come from the full design record and are NOT defined here.
//!
//! `Layout` is therefore a validated, backend-agnostic container of generic
//! `key = scalar` entries attached to one component. It carries no renderer,
//! platform, unit, or backend concepts, and assigns no meaning to entry
//! names. Structural validation only: identifier keys, scalar values, no
//! duplicates, no imperative operations, source-located Luca errors.

use crate::{LucaError, Prop};

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
        Ok(Self { entries, line, column })
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
