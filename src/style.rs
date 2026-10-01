//! Platform-independent styling representation for 0.3 Alpha.
//!
//! Locked facts from `docs/SPEC.md`: styling is part of the declarative
//! layer; the UI API is platform-independent and backed by a rendering
//! abstraction; no platform-specific components in the core vocabulary.
//! Nothing else about styling is locked: style property names, theme
//! tokens, defaults, and precedence rules must come from the full design
//! record and are NOT defined here.
//!
//! `Style` is therefore a validated, backend-agnostic container of generic
//! `key = scalar` entries attached to one component. It carries no renderer,
//! platform, theme-token, or precedence concepts, and assigns no meaning to
//! entry names. Structural validation only: identifier keys, scalar values,
//! no duplicates, no imperative operations, source-located Luca errors.

use crate::{layout::check_block_entries, LucaError, Prop};

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
        Ok(Self { entries, line, column })
    }

    /// True when the block declares no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
